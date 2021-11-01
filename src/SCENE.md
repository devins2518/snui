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

    The idea is to use **WidgetShell** to communicate the information about the context to widgets.
	It will reduce the burden of the **Context**.
	
- Make **Widget**s aware they are wrapped.
 
	This way they can delegate the drawing to their parents which is the **WidgetShell**
	
- Merge _input regions_ and _damage regions_.
 
	Their nature should be a property of the **Region** not different fields of the **Context**.
	
### 31 October

I removed the _input region_ field from the **Context** because its behaviour was inconsistent.
I then implemented my first and second ideas by adding a _wrapped_ field to the **Context** widgets can look 
to know if they're are wrapped in a **WidgetShell**. This addresses background artifacts partially but
not totally because there's not yet a way to set the background color of a parent and propagate it to it's transparent childs.

My ideas regarding how to solve that are:

	- add some kind of *background* field to the **Context**. **WidgetShell** could check it on a roundtrip
	- not rely on the **WidgetShell** and dynamically use the *background* field to draw a background on widgets with no background 

I'm more in favor of the second approach however this means I would have to scrap the first idea I implemented. Namely, 
> Make all **Container**s wrap their childs in a **WidgetShell**

