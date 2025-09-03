# Examples & Templates

Curated examples to validate different scene types and patterns (inspired by MAAACK scenes), plus language templates with the project’s commenting style.

## Contents

Load any scene in Godot to validate parsing, signal wiring, and structure checks.

#EOF

Templates (scenes/ui/components)
- templates/components/HealthNode.tscn: Wraps Indie Blueprint RPG health into a reusable scene.
- templates/ui/HUD.tscn (+ hud.gd): Simple HUD with HealthBar and message log; bind health_path to a Health node.
- templates/scenes/TurnBasedBattle.tscn: Skeleton turn-based battle wiring TurnityManager, two TurnitySockets, and HUD + Health for each side.

Usage
- Instance these scenes into your game scenes and adjust exported properties.
- For HUD, set the exported health_path to your Health node instance.