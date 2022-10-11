
# Slots/Processing/Containers

## Slots
- Deposit into a specific area and it will get slotted into an ascending* order
  through all the slots assigned to the deposit area.
- Spring forces keep the item in position, when the item has a spring force that
  is too great then it breaks free.

* Ascending is arbitrary and mostly just meant as an ordering behavior
  I think ideally for most things bottom up makes the most sense.

## Processing
- Once processing has begun slots are locked until it is finished processing,
  This should prevent accidentally flinging the items out of the slots while processing. Alternatively we could just have a special "SLOT_GROUPING" physics
  layer so that only the players hands can grab them out of the slots.
- Processing occurs on a slot level
- Once processing finishes, the item stays in the slot it is currently. If the 
  processing involves combining the items then the items move to the lowest slot
  in the ordering.

## Containers
- Non-processing slots, just for storage of ingredients
- Crate
    - 8 slots

## Processors
- Cauldron
    - 3 slots
    - Mixing

- Pestle & Mortar
    - 1 slot
    - Crushing
