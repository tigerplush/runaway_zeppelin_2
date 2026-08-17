# Vision

Runaway Zeppelin is a roguelike exploration simulator. The player is in control of a zeppelin on its first transatlantic flight, when the zeppelin is sucked into a storm and finds itself in a strange world. From here on out, it is the goal to survive and find the way back home. Surviving is done by managing resources and exploring [[Point of Interest|Points of Interest]]. At these POIs, the player can start expeditions and gain or lose resources or even new NPCs.

The core game loop consists of:
* [[Reach an event]]
* [[Plan and send out expedition]]
* [[Upgrade zeppelin]]

Furthermore, every [[NPC]] on board will be simulated. To change the behavior of [[NPC]]s, [[Policy|Policies]] can be enacted from the Pursers Cabin. The game is won when the Zeppelin reaches the Exit [[Event]]. Between play sessions, the money earned from cargo and passengers can be used to buy the  [[Meta Progression Updates]] for the Zeppelin.
The game ends when either the Zeppelin is destroyed, all [[NPC]]s die, the [[Morale]] sinks to 0 or the player finds and reaches the [[Storm]].

Unmentioned points:
* add different "factions" (e.g. inhabitants of the strange lands)
* buy maps that uncover/reveal parts of the world map

session:
* in-game: days to weeks