## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/ui/ui_example.gd                                     ┃
## ┃ Purpose: UI scene with signal to validate connections and layout    ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Control

@onready var _button: Button = %Button
@onready var _label: Label = %Label

## _ready
## Connects button signal and sets initial label text.
func _ready() -> void:
    # Debug: announce UI ready
    print("[Examples][UI] UIRoot ready")
    _button.pressed.connect(_on_button_pressed)

## _on_button_pressed
## Toggles label text to validate signal wiring and state changes.
func _on_button_pressed() -> void:
    # ── Toggle Phase ────────────────────────────────────────────────────
    _label.text = ("Clicked!" if _label.text != "Clicked!" else "Press the button")

#EOF