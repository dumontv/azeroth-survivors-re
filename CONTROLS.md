# Controls

Azeroth Survivors starts in third person with the WoW 1.12 interface and controller.
The original top-down view remains available, and your choice is saved locally.

| When | Key / Input | Action |
|---|---|---|
| Menu | Click a race, gender, class and zone | select a hero and arena |
| Menu | **Enter** / **Enter World** | start the run |
| Third person | **W / S** | run forward / backpedal |
| Third person | **A / D** | turn left / right |
| Third person | **Q / E** | strafe left / right |
| Third person | **right mouse drag** | turn the hero and camera together; A/D strafe while held |
| Third person | **left mouse drag** | orbit the camera without turning the hero |
| Third person | **both mouse buttons** | move forward |
| Top-down | **WASD** / **arrow keys** | move on the original fixed axes, facing the cursor |
| Playing | **Space** | jump |
| Playing | **mouse wheel** | zoom |
| Any | **F6** | switch third person / top-down; save the choice |
| Escape menu | **View** button | switch views |
| Playing | **F** (from level 5) | class movement spell: Blink, Charge, Sprint, etc. |
| Playing | **C / P** | open the stock character pane / spellbook |
| Playing | **Esc** | close the active WoW panel, or open the pause menu |
| Playing | — | survival abilities still cast automatically; action-bar icons show their cooldowns |
| Level up | **1 / 2 / 3** / click a card | pick an upgrade |
| Paused | **Esc** / **Return to Game** | resume |
| Paused | **Character selection** | return to the hero menu |
| Game over | **Enter** / **Play again** | return to the hero menu |
| Any | **G** | toggle picture polish |

The stock interface reads health, level, experience, learned survival spells, and their
cooldowns from the local run. The survival timer, kill count, and upgrade cards remain visible.
Character equipment, bags, quests, talents, and server-dependent UI actions are still awaiting
reconstruction; opening a stock panel does not supply those systems. Combat remains the fork's
automatic survival combat. F is its existing class mobility shortcut.

Custom view preferences live in `survivors-view.txt` under the fork's local configuration
folder, separately from stock WoW CVars. Removing that preference restores third person.
