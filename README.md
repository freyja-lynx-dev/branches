# Branches
This was going to be a GTK4/Relm4 based app for viewing raw ATProto data records. It was, in some ways, mostly there: while it wasn't pretty, it was able to pull records and create GTK widgets for it.

I ran into an issue that was squarely on me, I think -- I wanted a way of deduplicating records through a cache system (to remove unnecessary network requests if you're thumbing between a few records), which functioned in a multi-tab system. Designing the components around this was just... difficult. I bit off a bit more than I could chew.

I'm no longer that interested in working with GTK unfortunately. I feel like I could have designed a way out of this, but between the major issue I churned on for weeks and other things in my life I just wanted to let this one die.

I'm thinking about doing another project in the spirit of branches, but built with Dioxus or Leptos instead. That ecosystem seems not only just a bit more up my alley than Relm4 (which is a great project, FWIW), but also more applicable to the types of work I want to see myself doing.

If you wanna take over this project, feel free to fork it and comb through my mess :)
