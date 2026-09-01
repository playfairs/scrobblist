# Scrobblist

A cross-platform desktop client for Last.fm.

## Features

- **Profile viewing**: View your Last.fm profile information including username, playcount, and country
- **Recent scrobbles**: Browse your recently played tracks with timestamps
- **Now playing**: See your currently playing track when available

## Tech Stack

- **Backend**: Rust with Tauri 2
- **Frontend**: React with TypeScript
- **Styling**: Tailwind CSS
- **Database**: SQLite with SQLx
- **HTTP**: reqwest for Last.fm API calls
- **Authentication**: OS keychain via keyring crate

## Installation

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ with Cargo
- (Linux) Required system dependencies for Tauri

### Setup

1. Clone the repository:

```bash
git clone https://github.com/playfairs/scrobblist
cd scrobblist
```

2. Install frontend dependencies:

```bash
npm install
```

3. Set up environment variables:

```bash
cp .env.example .env
```

Edit `.env` and add your Last.fm API credentials:

```
LASTFM_API_KEY=your_api_key_here
LASTFM_API_SECRET=your_api_secret_here
```

To get Last.fm API credentials, sign up at [Last.fm API](https://www.last.fm/api/account/create).

For the callback URL, you must set it as `http://localhost:8080/callback`

## Development

### Running the application

Development mode with hot reload:

```bash
npm run tauri dev
```

### Building

Build for production:

```bash
npm run tauri build
```

This will create platform-specific bundles in the `src-tauri/target/release/bundle/` directory.

## Architecture

- **Frontend**: Handles presentation and UI state
- **Tauri Commands**: Bridge between frontend and Rust backend
- **Rust Backend**: Handles authentication, API calls, and database operations
- **SQLite**: Local cache for offline viewing and fast startup

## Acknowledgments

- Last.fm API: https://www.last.fm/api

---

This project is not affiliated with or endorsed by Last.fm. Last.fm is a trademark of Last.fm Ltd.
