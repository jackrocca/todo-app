#!/bin/bash

# Todo App Deployment Script
echo "🦀 Todo App Deployment Script"
echo "=============================="

# Build the application
echo "Building application..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

# Check if running with HTTPS
if [ "$USE_HTTPS" = "true" ]; then
    echo "🔒 HTTPS mode enabled"
    
    # Check for certificate files
    CERT_FILE="${CERT_PATH:-cert.pem}"
    KEY_FILE="${KEY_PATH:-key.pem}"
    
    if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
        echo "📜 Generating self-signed certificate for development..."
        openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"
        
        if [ $? -eq 0 ]; then
            echo "✅ Self-signed certificate generated"
            echo "   Certificate: cert.pem"
            echo "   Private Key: key.pem"
        else
            echo "❌ Failed to generate certificate"
            exit 1
        fi
    else
        echo "✅ Using existing certificates:"
        echo "   Certificate: $CERT_FILE"
        echo "   Private Key: $KEY_FILE"
    fi
    
    echo "🌐 Starting HTTPS server on port ${PORT:-3443}..."
    export USE_HTTPS=true
    export PORT=${PORT:-3443}
else
    echo "🌐 Starting HTTP server on port ${PORT:-3000}..."
    export USE_HTTPS=false
    export PORT=${PORT:-3000}
fi

# Set database URL if not provided
export DATABASE_URL=${DATABASE_URL:-"sqlite:todos.db"}

# Set JWT secret if not provided (use a secure random key in production)
export JWT_SECRET=${JWT_SECRET:-"your-secure-jwt-secret-change-in-production"}

echo "📊 Configuration:"
echo "   Database: $DATABASE_URL"
echo "   JWT Secret: [HIDDEN]"
echo "   HTTPS: $USE_HTTPS"
echo "   Port: ${PORT}"

# Run the application
echo "🚀 Starting Todo App..."
./target/release/todo-app