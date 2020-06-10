home-page = # Welcome to ncog.link

    ncog.link is aiming to be an online multiplayer game free-to-play platform targeting Windows, Mac OS X, Linux, iOS, and eventually Android. It aims to build high-level constructs that will enable event-driven scripting with easy-to-use multiplayer features. The project is in its infancy, and many details are still being worked out. If you like what you're reading, [please join the community](https://community.khonsulabs.com/) and help shape the future of ncog.

    Until there is a downloadable client to play around with, this page will serve as a bit of a README for the project. For the sake of this page, consider "me" to be [Jon aka @ecton](https://twitter.com/ectonDev). Eventually as this project grows, hopefully there will be more contributors, and it will make more sense to change the language accordingly.

    # Monetization

    Given how most free-to-play games are marketed, I wanted to place this at the top. ncog.link will only have one form of monetization: an optional subscription fee geared at supporting the platform and creators on the platform. There may also be physical merchandise or in-person events (once that becomes safe again), but in terms of digital purchases, the subscription fee will be the only source of revenue.

    There are some great examples where this works in today's society, the most predomenant is probably Costco. They align their own interests with their customers' interests by keeping their markups lower than their competitors, only stocking items that they believe are good value for their customers, and the way that they make a significant amount of that missing margin back is by selling their membership fees. If a Costco member no longer feels that they're getting enough value from Costco, they will leave. According to the 2019 annual report, Costco saw an impressive 88% worldwide renewal rate.

    In my opinion, this model should be the way that long-term creative edeavors are funded with, and that's why I'm happy to support many Patreon, OpenCollective, and Kickstarter.

    # Assets

    Through developing the platform, I will be commissioning some art sets to be built into the platform including a default set of Avatars. However, anyone can upload content they own, and any Cognitions that use those assets will automatically credit the creator.

    # Avatars

    Players will be able to create one or more avatars that they can use within various Cognitions. The avatar is just a name and a visual style. These avatars will be able to socialize with other avatars. If you're ever feeling like playing but not wanting to socialize, use the ncognito feature to play the game using a temporary randomized profile -- anything you collect will still be part of your normal avatar inventory.

    # Cognitions

    Cognitions are the individual "games" or environments that players will explore. Each Cognition can use any public assets on the platform, or can contain their own assets. If Cognitions need players to have different attributes, the player will need to create a new character or pick an existing one that fits the requirements. The goal of this design is to allow similar rulesets to be shared between Cognitions, not unlike how Dungeons & Dragons Editions have their own character sheet rules.

    A Cognition is guaranteed to have a map known as the Lobby, but beyond that, each Cognition will be unique.

    # Maps

    A map is a 2d graphics layer that can be painted using manually placed sprites or using a tile grid. Objects, boundaries, NPCs, Enemies, and more can be placed within the map. Each entity can have custom events tied to them that execute on the ncog server, and the results of each action are relayed to all clients connected to the map.

    Maps do not have to be defined statically, except for the Lounge. This means that one map could programmatically create a new map. In this way, you could have mechanism in game where two people sat on opposite sides of a table, and when two people were seated their characters were transfered to a new map which is a 1 vs 1 game. People who walked up to the table could spectate and would be transported to an area that allowed them to see what was going on without being able to interact. Once the game finishes, the map can kick everyone back out to lobby, and new people could sit and start the process again.

    # Server-driven User Interface

    Each Cognition may have a unique visual system that they need to convey information to the player via. The simplest example would be a health bar. Each Cognition can register events to be able to drive areas of the user interface, as well as present dialogs, or transition between animated scenes.

    # Music

    Early on, full tracks will be able to be hosted within the platform, and there will be an easy way to identify what background music is playing so that you can go see the track on other services like SoundCloud/BandCamp/Spotify/etc.

    Long term, creating a collaborative chiptune composition engine to create new tracks from scratch sounds fun to write, and it aligns several of my personal passions.

    # What does the path to getting this done look like?

    This is a massive project. I have a lot of experience building big systems, so I'm not really intimidated by this type of project. My initial goal is going to be getting the service set up with a single Cognition that allows people to walk around and interact with each other, and have some form of multiplayer game within that Cognition that visitors can play.

    By the time I reach that milestone, I will have a better idea of what the next milestone will look like. And if you want to help shape that vision, join up [on the forums](https://community.khonsulabs.com/) and have a chat!
