---

title: Make the Change Easy, Then Make the Easy Change
subtitle: Refactoring
author: Corey Alexander
date: 2024-06-26

---

A manta I’ve repeated for a while in my software career has been “Make the Change Easy, Then make the Easy Change”.

This phrase is about adding new features or options to your code, but what it really gets at is the constant need to refactor your code to make them amendable to changes.

When you go to add a new feature to your codebase, you are often faced with a decision:

- Hack in the needed functionality to the existing structure
- Refactor some stuff to make the new feature fit nicely

Me and this mantra are advocating for the second option here. And that’s not to say that there aren’t times when the first “hacky” option is the right choice. But more often than not doing one hacky thing is a slippery slope to continuing to hack in new features, since each time you are in those code paths they are worse than the time before.

By refactoring BEFORE you add a new feature your codebase is getting cleaner and cleaner as you go! As opposed to creating more tech debt with each PR, you can actually clean up your codebase as you go! This will pay for itself in the long run, AND has the benefit of making the work more “predictable”. In a clean, well organized code base there are less chances of running into something unexpected that drastically shifts the timelines for a project.
In a messy codebase it’s common to find issues buried, or things that take longer to untangle than expected.

Not only does cleaning up the codebase allow you to be more productive, it also helps be more predictable! And predictability is sometimes even more important than raw speed. Something being finished a day or two earlier is often not a deal breaker, but missing an agreed upon deadline because something unexpected came up can cause issues for the organization.

I'm a firm believer that in 90% of circumstances its better to refactor the code ahead of an expected change, instead of hacking it in and just kicking the cleanup can down the road.
