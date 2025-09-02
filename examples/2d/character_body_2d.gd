## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/2d/character_body_2d.gd                              ┃
## ┃ Purpose: Minimal CharacterBody2D movement example for validation     ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends CharacterBody2D

## Movement configuration — tuned for predictable validation
const SPEED: float = 180.0  # pixels/sec
const ACCEL: float = 700.0  # pixels/sec^2
const FRICTION: float = 900.0  # pixels/sec^2

var _input_vec: Vector2 = Vector2.ZERO  # cached input direction

## _ready
## Ensures node is bootable and prints a debug line for tooling.
func _ready() -> void:
    # Debug: announce scene is ready; used by scene monitor and tests
    print("[Examples][2D] CharacterBody2D ready")

## _physics_process
## Integrates velocity using simple acceleration/friction model.
## Edge cases: diagonal movement normalized; zero-input applies friction.
func _physics_process(delta: float) -> void:
    # ── Input Phase ─────────────────────────────────────────────────────
    _input_vec = Input.get_vector("ui_left", "ui_right", "ui_up", "ui_down")

    # ── Velocity Solve ──────────────────────────────────────────────────
    var target_vel: Vector2 = _input_vec * SPEED
    var diff: Vector2 = target_vel - velocity

    if _input_vec != Vector2.ZERO:
        # Accelerate toward target
        var step: Vector2 = diff.clamp(Vector2(-ACCEL * delta, -ACCEL * delta), Vector2(ACCEL * delta, ACCEL * delta))
        velocity += step
    else:
        # Apply friction when idle
        velocity = velocity.move_toward(Vector2.ZERO, FRICTION * delta)

    # ── Move ────────────────────────────────────────────────────────────
    move_and_slide()

#EOF