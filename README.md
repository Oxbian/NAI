# NAI
---

Néo AI, a personnal assistant using LLM.
A TUI interface for local llama.cpp LLM, in the future more functionnality will
be added to this AI.

## Usage

### Dependencies

This project is written in Rust, so you will need `rustc` and `cargo`.  
Moreover, you will need a LLM API, currently only works with local 
[llama.cpp](https://github.com/ggml-org/llama.cpp) API. 
  
### Building & Running

To build and run this project you will need to install all the dependencies used:
  
```bash
cargo install
```
  
Once that is done, just 
```bash
cargo run
``` 
and there you go !

## Screenshots

![Screenshot of the ui](screenshots/ui.png)

## TODO

- Color change if it's an user or the LLM
- Async request to the LLM API
- Start the real fun
