# Text2Deck

A modern web application that converts text content into Google Slides presentations automatically, built with Rust and deployed on Cloudflare Workers.

## Architecture

- **Backend**: Cloudflare Worker (Rust) - API and OAuth handling
- **Frontend**: Yew (Rust + WASM) - Modern web interface with Tailwind CSS
- **Storage**: Cloudflare KV - Session token storage
- **APIs**: Google Slides API, Google Drive API

## Features

- 🔐 **Secure OAuth 2.0** authentication with Google
- 📄 **Multiple text splitting strategies**:
  - Split by lines
  - Split by paragraphs (empty lines)
  - Split by maximum word count
  - Split by maximum character count
- 🎨 **Automatic Google Slides creation** with proper formatting
- 🌐 **Modern web interface** built with Yew and Tailwind CSS
- ⚡ **Fast and responsive** - WASM-powered frontend
- 🔒 **Secure session management** with HttpOnly cookies

## Setup

### 1. Prerequisites

- Cloudflare Workers account
- Google Cloud Console project with Slides API enabled
- Rust toolchain with wasm-pack

### 2. Google Cloud Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing one
3. Enable the Google Slides API and Google Drive API
4. Create OAuth 2.0 credentials:
   - Go to APIs & Services > Credentials
   - Click "Create Credentials" > "OAuth 2.0 Client ID"
   - Application type: Web application
   - Add authorized redirect URI: `https://your-worker-domain.workers.dev/oauth/callback`
5. Note down the Client ID and Client Secret

### 3. Cloudflare Workers Setup

1. Install Wrangler CLI:

   ```bash
   npm install -g wrangler
   ```

2. Login to Cloudflare:

   ```bash
   wrangler login
   ```

3. Set up environment variables:

   ```bash
   wrangler secret put GOOGLE_CLIENT_ID
   wrangler secret put GOOGLE_CLIENT_SECRET
   wrangler secret put GOOGLE_REDIRECT_URI
   ```

4. Create a KV namespace for storing tokens:

   ```bash
   wrangler kv:namespace create "TOKENS"
   ```

   Add the returned binding to your `wrangler.toml`.

### 4. Deploy

```bash
wrangler deploy
```

## Usage

### Web Interface

1. Navigate to `https://your-worker-domain.workers.dev/app`
2. Click "Authenticate with Google"
3. Enter your text content
4. Choose a splitting method
5. Click "Create Slides"

### API Endpoints

#### Authentication

- `GET /oauth/start` - Start OAuth flow
- `GET /oauth/callback` - OAuth callback handler

#### Slides Creation

- `POST /api/create-slides` - Create slides from text

Request body:

```json
{
  "title": "My Presentation",
  "content": "Your text content here...",
  "splitter_type": "newline",
  "splitter_config": {
    "max_words": 50,
    "max_chars": 500
  }
}
```

Response:

```json
{
  "presentation_id": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms",
  "presentation_url": "https://docs.google.com/presentation/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms/edit",
  "message": "Slides created successfully"
}
```

#### Utility

- `GET /api/splitters` - Get available splitter types
- `GET /health` - Health check

## Splitter Types

### `newline`

Splits text by individual lines. Each line becomes a slide.

### `empty_line`

Splits text by paragraphs (separated by empty lines).

### `max_words`

Splits text by maximum word count per slide.

- Config: `max_words` (default: 50)

### `max_chars`

Splits text by maximum character count per slide.

- Config: `max_chars` (default: 500)

## Development

### Local Development

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check code
cargo clippy
```

### Project Structure

```
├── worker/              # Cloudflare Worker (Rust)
│   ├── src/
│   │   ├── lib.rs       # Main worker entry point
│   │   ├── oauth.rs     # OAuth 2.0 handling
│   │   ├── slides.rs    # Google Slides API integration
│   │   ├── splitter.rs  # Text splitting strategies
│   │   └── error.rs     # Error handling
│   ├── wrangler.toml    # Cloudflare Workers configuration
│   └── Cargo.toml       # Worker dependencies
├── web/                 # Web Frontend (Yew + WASM)
│   ├── src/
│   │   ├── lib.rs       # Entry point and app setup
│   │   ├── types.rs     # Type definitions
│   │   ├── api.rs       # API communication
│   │   └── components/  # Yew components
│   │       ├── app.rs           # Main app component
│   │       ├── auth_section.rs  # Authentication UI
│   │       ├── slides_form.rs   # Slides creation form
│   │       └── status_message.rs # Status/error messages
│   ├── index.html       # HTML entry point
│   └── Cargo.toml       # Frontend dependencies
└── README.md            # This file
```

## Quick Start

### 1. Frontend Development

```bash
cd web
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8000
# Open http://localhost:8000
```

### 2. Worker Development

```bash
cd worker
cargo test
wrangler dev  # For local development
```

## Security

- OAuth 2.0 with PKCE for secure authentication
- Session tokens stored in Cloudflare KV with expiration
- HttpOnly, Secure cookies
- CSRF protection via state parameter

## Limitations

- Maximum presentation size depends on Google Slides API limits
- Token refresh not implemented (tokens expire after ~1 hour)
- Limited slide layouts (uses default title and body layout)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
