extends Node

export (PackedScene) var gui_scene
export (float) var toggle_delay
onready var gui_container = $Container
var timer: Timer

# This script tests for memory leaks in GodotEgui by repeatedly instancing and freeing a scene.
# In order to replicate the results, you should run the scene, open the debugger and check the memory monitor.
# If memory usage stays stable, it means everything is working as intended.
func _ready():
	var gui = self.gui_scene.instance()
	self.add_child(gui)
	self.timer = Timer.new()
	self.timer.autostart = true
	self.timer.wait_time = self.toggle_delay
	self.add_child(timer)
	self.timer.one_shot = false

	var err = timer.connect("timeout", self, "_on_timeout")
	assert(err == OK)

func _on_timeout():
	print("timeout completed")
	if self.gui_container.get_child_count() == 0:
		print("instance scene")
		var gui = self.gui_scene.instance()
		gui_container.add_child(gui)
	else:
		for child in gui_container.get_children():
			print("free scene")
			child.queue_free()
#	self.timer.start(1)
