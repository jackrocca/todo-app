use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

pub struct AuthService {
    pool: SqlitePool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // Check if user exists
        let existing_user = sqlx::query(
            "SELECT id FROM users WHERE username = ?1 OR email = ?2",
            req.username,
            req.email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::DatabaseError)?;

        if existing_user.is_some() {
            return Err(AuthError::UserExists);
        }

        // Hash password
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::HashError)?;

        // Create user
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let user = sqlx::query_as(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING id, username, email, password_hash, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            "#,
            id, req.username, req.email, password_hash, now, now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::DatabaseError)?;

        let token = self.generate_token(&user.id)?;

        Ok(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            },
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AuthError> {
        let user = sqlx::query_as(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            FROM users WHERE username = ?1
            "#,
            req.username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::DatabaseError)?
        .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let password_valid = bcrypt::verify(&req.password, &user.password_hash)
            .map_err(|_| AuthError::HashError)?;

        if !password_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let token = self.generate_token(&user.id)?;

        Ok(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            },
        })
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, AuthError> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| AuthError::TokenError)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::TokenError)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            FROM users WHERE id = ?1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::DatabaseError)?;

        Ok(user)
    }
}

#[derive(Debug)]
pub enum AuthError {
    DatabaseError,
    UserExists,
    InvalidCredentials,
    HashError,
    TokenError,
    Unauthorized,
}

impl From<AuthError> for StatusCode {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::UserExists => StatusCode::CONFLICT,
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::HashError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::TokenError => StatusCode::UNAUTHORIZED,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

// Middleware for JWT authentication
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = auth_service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user ID to request extensions
    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}