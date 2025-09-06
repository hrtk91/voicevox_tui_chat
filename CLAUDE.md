# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running

- `cargo run` - Build and run the chat terminal application
- `cargo build` - Build the project
- `cargo check` - Fast compilation check without code generation

### Code Quality

- `cargo fmt` - Format Rust code
- `cargo clippy` - Run linting checks

## Environment Setup

The application requires a `.env` file in the root directory with the following variables:

- `OPENAI_API_KEY` - OpenAI API key for chat completions
- `VOICEVOX_ENGINE_URL` - URL to running VOICEVOX Engine (default: http://localhost:50021)
- `OPENAI_MODEL` - OpenAI model to use (options: gpt-5, gpt-5-mini, gpt-5-nano; default: gpt-5-nano)
- `SYSTEM_PROMPT` - System prompt for the AI (optional)

VOICEVOX Engine must be running separately. The README suggests using Docker:
`docker run --rm -d -p 50021:50021 --gpus all voicevox/voicevox_engine`

## Architecture Overview

This is a terminal-based AI chat application with voice synthesis capabilities, built using Rust's async/await and a feature-based architecture.

### Core Architecture

**Feature-based Module Structure:**

- `src/features/chat/` - Chat functionality (state management, UI rendering, OpenAI integration)
- `src/features/terminal/` - Terminal UI application loop and event handling
- `src/features/voice/` - Voice synthesis integration with VOICEVOX

**Key Components:**

1. **Terminal App (`src/features/terminal/app.rs`)**

   - Main application loop using ratatui for TUI
   - Handles keyboard input and coordinates between features
   - Event-driven architecture processing `ChatEvent`s from worker

2. **Chat System**

   - **Worker (`src/features/chat/worker.rs`)**: Async background task handling OpenAI API streaming
   - **State (`src/features/chat/state.rs`)**: Message management, scroll state, input modes
   - **Events (`src/features/chat/events.rs`)**: Event definitions and handlers for chat interactions
   - **Components (`src/features/chat/components.rs`)**: UI rendering logic

3. **Voice Integration (`src/features/voice.rs`)**
   - Listens for `ChatEvent::StreamingComplete` events
   - Automatically synthesizes AI responses using VOICEVOX API
   - Integrates existing `audio.rs` (VOICEVOX API) and `sound.rs` (audio playback)

### Event Flow

1. User input → `ChatWorker` → OpenAI API streaming
2. Streaming chunks → `ChatEvent::StreamingChunk` → UI updates
3. Stream complete → `ChatEvent::StreamingComplete` → Voice synthesis + UI finalization

### Audio Pipeline

- `src/audio.rs` - VOICEVOX API integration (text → WAV generation)
- `src/sound.rs` - Audio playback using rodio
- `src/features/voice.rs` - Orchestrates text-to-speech pipeline

### OpenAI Integration

- `src/openai.rs` - Streaming chat completions with conversation history
- Message history maintained in `ChatWorker` for context continuity

## Key Patterns

- **Async Event-Driven**: Uses tokio mpsc channels for communication between terminal UI and background workers
- **Feature Isolation**: Each feature handles its own events independently
- **Streaming UI**: Real-time display of AI responses as they're generated
- **Non-blocking Voice**: Voice synthesis runs in background tokio tasks to avoid UI freezing
