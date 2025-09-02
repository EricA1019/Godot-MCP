## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: examples/autoload/Global.gd                                   ┃
## ┃ Purpose: Autoload singleton example for validation                  ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Node

## Global state used by examples and tests
var counter: int = 0

## inc
## Increments global counter and logs value.
func inc() -> void:
    counter += 1
    print("[Examples][Autoload] counter=", counter)

#EOF