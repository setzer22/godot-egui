tool
extends EditorPlugin

# A class member to hold the dock during the plugin life cycle.
const Dock = preload("res://GodotEguiExample.tscn")
var dock_instance


func _enter_tree():
    dock_instance = Dock.instance()
    # Add the main panel to the editor's main viewport.
    add_control_to_dock(DOCK_SLOT_LEFT_UL, dock_instance)

		
func _exit_tree():
    # Clean-up of the plugin goes here.
    # Remove the dock.
    remove_control_from_docks(dock_instance)
    # Erase the control from the memory.
    dock_instance.free()
