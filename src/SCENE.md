# SCENE

Introducing scenes into snui.

### What is a scene?

The scene is a simpler representation of the widget or the application states.

### Why use a scene?

When damaging it is inevitable that you'll redraw over an existing widget but you don't want to see the previous
version of the widget under its most recent update. The erase the previous image, you need to overwrite that canvas
but in order to do so, you need to know what was is under the widget. The scene serves to provide that contextual information.

### My ideas so far

- Make all **Container**s wrap their childs in a **WidgetShell**
	The idea is to use WidgetShell to communicate the information about the context to Widgets.
	It will reduce the burden of the Context.
- Make **Widget**s aware they are wrapped.
	This way they can delegate the drawing to their parents which is the WidgetShell
- Merge _input regions_ and _damage regions_.
	Their nature should be a property of the **Region** not different fields of the Context.
