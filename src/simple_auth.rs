use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
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
    pub user_id: String,
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
        let existing = sqlx::query("SELECT id FROM users WHERE username = ? OR email = ?")
            .bind(&req.username)
            .bind(&req.email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

        if existing.is_some() {
            return Err(AuthError::UserExists);
        }

        // Hash password
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::HashError)?;

        // Create user
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&req.username)
            .bind(&req.email)
            .bind(&password_hash)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

        let token = self.create_token(&id)?;
        Ok(AuthResponse { token, user_id: id })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AuthError> {
        let row = sqlx::query("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&req.username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        let user_id: String = row.get("id");
        let stored_hash: String = row.get("password_hash");

        let valid = bcrypt::verify(&req.password, &stored_hash)
            .map_err(|_| AuthError::HashError)?;

        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let token = self.create_token(&user_id)?;
        Ok(AuthResponse { token, user_id })
    }

    fn create_token(&self, user_id: &str) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_ref()))
            .map_err(|_| AuthError::TokenError)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>, AuthError> {
        let row = sqlx::query("SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    fn decode_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &DecodingKey::from_secret(self.jwt_secret.as_ref()), &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }
}

pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let Some(token) = auth_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let claims = auth_service.decode_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(claims.sub);

    Ok(next.run(request).await)
}

#[derive(Debug)]
pub enum AuthError {
    DatabaseError,
    UserExists,
    InvalidCredentials,
    HashError,
    TokenError,
    InvalidToken,
}

impl From<AuthError> for StatusCode {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::UserExists => StatusCode::CONFLICT,
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::HashError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::TokenError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
        }
    }
}