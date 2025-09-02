## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/physics2d/physics2d_example.gd                      ┃
## ┃ Purpose: Simple physics scene to validate Areas, Bodies, signals   ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Node2D

@onready var _area: Area2D = $Area2D
@onready var _body: RigidBody2D = $Body

## _ready
## Connects signals and applies an initial impulse.
func _ready() -> void:
    # Debug: connect signals to validate scene tracing
    _area.body_entered.connect(_on_area_body_entered)
    _body.apply_impulse(Vector2.RIGHT * 100.0)

## _on_area_body_entered
## Logs body entry into area; used for signal validator tests.
func _on_area_body_entered(body: Node) -> void:
    print("[Examples][Physics2D] body entered area:", body.name)

#EOF