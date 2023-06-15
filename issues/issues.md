
# Slots
- Slots do not apply an impulse/torque to the container, ideally it should be applied on the rigid body it is attached to.

# Grabbing
- Grabbing an object now allows you to fly, probably because the anchor of the hand is now the center of the hand.
- Intuitive grabbing rotation is tricky
  - Maybe should scrap and just have auto aim helpers for things like the stirrer/mortar & pestle?
  - Grab sphere needs to be relative to the player.
  - Feedback loops must be avoided, otherwise small errors will creep in over time. Therefore the player should have a "target position" for the hands separated from reality.