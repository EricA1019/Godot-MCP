## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/3d/node3d_example.gd                                 ┃
## ┃ Purpose: Minimal 3D scene for validation and monitoring             ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Node3D

## Rotation speed in degrees per second
const ROT_SPEED: float = 30.0

## _process
## Rotates the child MeshInstance3D to provide a visible, testable change
func _process(delta: float) -> void:
    # Debug: visible rotation for scene monitor
    rotate_y(deg_to_rad(ROT_SPEED * delta))

#EOF