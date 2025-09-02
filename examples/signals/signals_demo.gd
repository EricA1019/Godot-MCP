## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/signals/signals_demo.gd                              ┃
## ┃ Purpose: Demonstrate custom signal emission and receipt             ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Node

signal ping(payload: String)

@onready var _emitter: Node = $Emitter
@onready var _receiver: Node = $Receiver

## _ready
## Connects custom signal and fires an initial test ping.
func _ready() -> void:
    # Debug: connect and send initial ping
    self.ping.connect(_on_ping)
    # Touch onready nodes for validation to avoid unused warnings
    _emitter.set_meta("role", "emitter")
    _receiver.set_meta("role", "receiver")
    emit_signal("ping", "hello")

## _on_ping
## Receives custom signal; used to validate signal tracing.
func _on_ping(payload: String) -> void:
    # ── Receive Phase ───────────────────────────────────────────────────
    print("[Examples][Signals] ping=", payload)

#EOF