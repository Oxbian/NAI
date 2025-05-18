# NAI
---

Néo AI, a personnal assistant using LLM.
A TUI interface for local llama.cpp LLM, in the future more functionnality will
be added to this AI.

> [!CAUTION]
> This project is designed to help me understand modern techniques around AI and
> LLM. Other projects like [txtchat](https://github.com/neuml/txtchat) and
> [LangRoid](https://langroid.github.io/langroid/) are more advanced and better for real use.

## Usage

### Dependencies

This project is written in Rust, so you will need `rustc` and `cargo`.  
- A [ollama](https://github.com/ollama/ollama) API for LLM
- A [kiwix server](https://kiwix.org/en/applications/) web server and API for getting data from a zim
  file (wikipedia).

### Building & Running

To build and run this project you will need to install all the dependencies used:
  
```bash
cargo install
```
  
Once that is done, just 
```bash
cargo run
``` 
and there you go for the UI !
  
Ollama must be running, a model (like llama3.2) must be downloaded.
Configuration files will probably need some modifications, so check everyone of
them.  
  
For Wikipedia data, run a kiwix webserver on your zim file and modify the
configuration in `wiki/wiki.json`.
  
```bash
kiwix-serve -p 1025 -i 127.0.0.1 MYZIMFILE.zim
```

## Screenshots

![Screenshot of the ui](screenshots/ui.png)

## Feature

- Conversation are saved inside files in JSON in this folder `conv/`, and can be reused on others LLM.
- In normal mode, conversation can be resumed by the LLM into bullet point list.
- LLM can be configured thanks to configuration files in `config/`
- Requests are routed thanks to the LLM to other expert LLM. Code questions are send to a code expert, wikipedia questions are send to a module which use a kiwix API to provide data from Wikipedia.

## TODO

- Connect & try LLM / tools

## Inspiration and reason

Why in Rust ?

Because I wanted to learn Rust, and the language is fast and powerful.

- [WilmerAI](https://github.com/SomeOddCodeGuy/WilmerAI/) a system where all the inferences are routed to others expert LLM.
