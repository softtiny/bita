#!/usr/bin/env just --justfile
set dotenv-load
set dotenv-filename := ".env.local"
default:
  just --list
reqwest_proxy__server_proxy:
    cargo test --package bitar --features rustls-tls --test reqwest_hyper_proxy request_server_proxy -- --nocapture