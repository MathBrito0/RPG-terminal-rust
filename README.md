![Shujinko vs Chelebrimbor - Powered by Rust](./assets/rpg_rust.jpg)

# ⚔️ Shujinko vs Chelebrimbor: Terminal RPG

> A modular turn-based RPG developed in **Rust**, combining rich, dynamic inventory mechanics with a modern Terminal User Interface (TUI) built using the **Ratatui** library.

---

## 📸 About the Project

This project represents an evolution from a traditional text-based RPG into a polished, interactive terminal application. The player takes on the role of **Shujinko** (a fearless Mage) in a relentless tactical duel against the elf **Chelebrimbor**, managing resources, potions, spells, and defenses in real-time.

> [!NOTE]
> I acknowledge the use of the "Google Gemini Auto (2026)" LLM model to assist in the development of certain parts of this project—specifically regarding styling (using Ratatui) and code debugging.

---

## ✨ Key Features

* **Modern TUI (`Ratatui` & `Crossterm`):** Replaces scrolling text with a fixed layout organized into aesthetic blocks (Status Panel, Event History, and Action Menu).
* **Tactical Turn-Based Combat:** Choose between attacking with a weapon, casting spells, defending (with actual damage reduction for the following turn), or calling upon your pet/companion.
* **Dynamic and Safe Inventory:** An interactive, indexed backpack that allows for the use of healing, magic, and stamina potions, with safe boundary checks (using `saturating_sub`).
* **Modular Architecture:** Clean, decoupled code organized into distinct modules for the Hero, the Enemy, and the core game logic. ---

## 🛠️ Technologies Used

* **[Rust](https://www.rust-lang.org/)** - A programming language focused on safety and high performance.
* **[Ratatui](https://github.com/ratatui/ratatui)** - A Rust library for building terminal user interfaces (TUIs).
* **[Crossterm](https://github.com/crossterm-rs/crossterm)** - A cross-platform library for terminal manipulation and keyboard event handling.

---

## 📂 Project Structure

```text
├── Cargo.toml          # Project dependencies and metadata
├── .gitignore          # Files and folders ignored by Git (/target)
└── src/
├── main.rs         # Main loop, TUI state management, and event handling
├── heroi.rs        # Character structs, inventory, and potion rules
└── inimigo.rs      # Opponent (Chelebrimbor) structs and attributes
```
