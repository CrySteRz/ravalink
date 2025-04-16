# Ravalink

**A Kafka-Powered Lavalink Alternative for Discord Bots**

## Overview
Ravalink is a scalable audio processing service for Discord bots, designed as a Lavalink alternative with exclusive Kafka integration. It handles audio playback, track scheduling, and event processing via Kafka, requiring coordination with `ravalink-lib` (bot-side library) and `ravalink-interconnect` (message specification).

## Features
- **Kafka-Centric Architecture**: All audio events and commands are processed through Kafka.
- **Horizontal Scalability**: Deploy multiple instances using Kafka for load distribution.
- **Interoperability**: Works with `ravalink-lib` (bot SDK) and `ravalink-interconnect` (message schema).
- **Low-Latency Audio**: Optimized for real-time Discord bot interactions.

## Prerequisites
- Rust 1.65+
- Apache Kafka
- [`ravalink-interconnect`](https://github.com/CrySteRz/ravalink-interconnect) (message schema)
