# NAI
---

Néo AI, a personnal assistant using LLM.
A TUI interface for local llama.cpp LLM, in the future more functionnality will
be added to this AI.

## Usage

### Dependencies

This project is written in Rust, so you will need `rustc` and `cargo`.  
Moreover, you will need a LLM API, currently only works with local 
[ollama](https://github.com/ollama/ollama) API. 
  
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

## Feature

- Conversation are saved inside files in JSON in this folder `conv/`, and can be reused on others LLM.
- In normal mode, conversation can be resumed by the LLM into bullet point list.
- LLM can be configured thanks to configuration files in `config/`
- Requests are routed thanks to the LLM to other expert LLM. Code questions are send to a code expert, wikipedia questions are send to a system which provide better factual response.

## TODO

- Color change if it's an user or the LLM
- Async request to the LLM API
- Start the real fun

## Inspiration and reason

Why in Rust ?

Because I wanted to learn Rust, and the language is fast and powerful.


- [WilmerAI](https://github.com/SomeOddCodeGuy/WilmerAI/) a system where all the inferences are routed to others expert LLM.
