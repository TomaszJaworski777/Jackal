<div align="center">

<img
  width="200"
  alt="Jackal Logo"
  src=".readme/logos/logo_rounded_corners.png">
 
<h3>Jackal</h3>
<b>Most aggressive CPU MCTS chess engine</b>
<br>
<br>

[![License](https://img.shields.io/github/license/TomaszJaworski777/Jackal?style=for-the-badge)](https://opensource.org/license/gpl-3-0)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/TomaszJaworski777/Jackal?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/releases/latest)
[![Commits](https://img.shields.io/github/commits-since/TomaszJaworski777/Jackal/latest?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/commits/main)
<br>
[![Lichess Bullet](https://lichess-shield.vercel.app/api?username=JackalBot&format=bullet)](https://lichess.org/@/JackalBot)
[![Lichess Blitz](https://lichess-shield.vercel.app/api?username=JackalBot&format=blitz)](https://lichess.org/@/JackalBot)
[![Lichess Rapid](https://lichess-shield.vercel.app/api?username=JackalBot&format=rapid)](https://lichess.org/@/JackalBot)
<br>
[![Build](https://img.shields.io/github/actions/workflow/status/TomaszJaworski777/Jackal/build.yml?branch=main&label=build)](https://github.com/TomaszJaworski777/Jackal/actions/workflows/build.yml)
[![Tests](https://img.shields.io/github/actions/workflow/status/TomaszJaworski777/Jackal/test.yml?branch=main&label=tests)](https://github.com/TomaszJaworski777/Jackal/actions/workflows/test.yml)
<br>

| Version | CCRL Blitz | CCRL 40/15 | CEGT 40/20 | EAS Score | Release Date |
| :-: | :-: | :-: | :-: | :-: | :-: |
| [2.0.0](https://github.com/TomaszJaworski777/Jackal/releases/tag/2.0.0) | 3485* | - | - | 403.35k | 5th March 2026 |
| [1.2.0](https://github.com/TomaszJaworski777/Jackal_old/releases/tag/1.2.0) | - | 3073 | 2984 | 131.02k | 20th December 2024 |
| [1.1.0](https://github.com/TomaszJaworski777/Jackal_old/releases/tag/1.1.0) | - | 2963 | - | 194.26k | 7th December 2024 |
| [1.0.0](https://github.com/TomaszJaworski777/Jackal_old/releases/tag/1.0.0) | 2956 | - | - | 229.96k | 4th December 2024 |

<i>* estimated in 40+0.4 gauntlet</i>

</div>

## Overview
Jackal is the chess engine written in Rust, that I'm currently developing as a follow-up to [Javelin](https://github.com/TomaszJaworski777/Javelin). It uses Monte Carlo Tree Search (MCTS) search algorithm, with a lot of improvements to the selection and expansion stages in order to achieve very aggressive style of play. The idea is to create something that consistently seeks out high-risk, high-reward positions. It uses my own move generator library, [Spear](https://github.com/TomaszJaworski777/Spear), capable of generating up to 750 million moves per second.

Jackal uses 7 neural networks in total, 3 for value and 4 for policy. Primary value and policy networks were trained through self play from noise. Secondary networks are finetuned versions of the primary and are used based on the general score of the position. For example, if the general score is very low, the engine will use the primary network, but as the score is getting higher, the engine will swap to secondary or tertiary network. This is done in order to make the engine more aggressive in winning positions and more defensive in losing positions. Raw outputs from the network are modified by several heuristics in order to bias the engine towards more volatile positions.

As a result Jackal is able to play far beyond superhuman level (almost 3500 elo) while maintaining very aggressive style of play as proved by its EAS score of 403k.

Archive of the previous repository of this engine can be found [here](https://github.com/TomaszJaworski777/Jackal_old).

## Selecting The Best Version
To choose the best version of the engine to download, check the [Microarchitecture levels table](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels). If you still are not sure, try x86-64-v3, it should run on most CPUs.

## Compiling
You can also compile the engine yourself optimized for your CPU by following the steps below:
1. Download the source code
2. Run `make` command in root folder
3. Binary called `jackal` should appear in the root folder

## EAS
Jackal 2.0.0 uses the new version of the EAS tool, which moves the scale of the EAS score. After a lot of effort  I still did not manage to get Jackal to avoid bad draws and finish games fast. The level of play and quality of sacs improved a lot though, so I'm happy with the result, even though it's a bit underwhelming on the EAS results.

#### New version of EAS tool table
| EAS Score| Sacs | Early Sacs | Shorts | Bad draws | Avg. win moves  | Version  |
| :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| 403348 | 30.15% | 62.99% | 12.81% | 16.09% | 92 | 2.0.0 |

#### Old table, including Jackal 2.0.0, but using the old EAS tool
| EAS Score| Sacs | Shorts | Bad draws | Avg. win moves  | Version  |
| :-: | :-: | :-: | :-: | :-: | :-: |
| 169095 | 30.19% | 11.49% | 16.07% | 92 | 2.0.0 |
| 131021 | 28.99% | 09.45% | 21.26% | 98 | 1.2.0 |
| 194262 | 34.56% | 16.64% | 13.91% | 75 | 1.1.0 |
| 229966 | 40.00% | 26.71% | 17.36% | 103 | 1.0.0 |

I implemented a lot of features to improve EAS score, here is a breakdown of the changes:
#### 7 Asymmetrical networks 
I trained strong value and policy networks. Then I filtered data using several different filters to achieve dataset of very aggressive positions where sac allowed the engine to win the game. Then I finetuned networks on this dataset using different learning rates to achieve different levels of aggression. The engine swaps between the networks based on the general score of the position.
#### Sac selection bonus
When the engine selects the best child node to report as its move, sacrificing moves get a flat bonus added to their score. The bonus scales with the material value of what was sacrificed and also grows in winning positions, the better Jackal is doing, the more it prefers to keep sacrificing.
#### Sac exploration bonus
Same idea but applied during MCTS selection stage. Sacrificing moves get an extra bonus on top of their exploration term, which makes the engine visit them more during search. This bonus also scales with winning probability above 0.51, and increases further when the position is strongly winning (above 0.75).
#### Policy sac bonus
When a new node is expanded, the policy probability of each move is modified before the softmax. Moves that fail SEE (moves that seemingly lose material) get a raw bonus proportional to the value of the piece being lost. This biases the policy output toward material sacrifices before the neural network scores are even applied.
#### Draw pessimism
A penalty is subtracted from the draw probability during simulation. In the opening and middlegame the penalty is larger, and it decreases as the game approaches an endgame. This effectively makes draws look slightly worse than they are, pushing the engine away from repetitions and equal trades.
#### Draw scaling (50MR scaling)
The WDL score is adjusted based on the half-move clock. As the position approaches the 50-move rule limit, more of the win/loss probability gets converted to draw probability. On top of that, deeper simulations also get a small additional draw push to account for uncertainty at depth.
#### Contempt
Applied after the simulation, this shifts the WDL on a logistic scale based on an estimated Elo advantage. When playing a weaker opponent, contempt is set high automatically, making draws look much worse and pushing the engine toward playing for a win instead of taking safe equal lines. Can be set manually via `UCI_RatingAdv`. Default value of contempt is 300, making Jackal a bit disrespectful by default. 
#### Proof Number Search
Proof number search tracks, for each node, how many leaf nodes need to be explored to prove a win. Jackal uses a one-sided version where only proof numbers are maintained, without disproof numbers. This is applied only when the engine is in a winning position, biasing the search toward lines that are closest to being fully proven, which hopefully helps convert advantages faster.
#### Sacrifice time manager
One of the modules in the time manager is dedicated to positions where sacrificial moves are among the 2 best moves according to the search.
## Engine Options

| Name | Default | Min | Max | Description |
| :-- | :-: | :-: | :-: | :-- |
| `Hash` | 32 | 1 | 524288 | Size of the search tree in MB. |
| `Threads` | 1 | 1 | 1024 | Number of search threads. |
| `MoveOverhead` | 10 | 0 | 2000 | Extra time buffer in ms subtracted from the time limit to avoid losing on time. |
| `MultiPV` | 1 | 1 | 218 | Number of best lines to search and report simultaneously. |
| `UCI_Chess960` | false | — | — | Enable Chess960 (Fischer Random) move parsing and output. |
| `UCI_ShowWDL` | false | — | — | Show Win/Draw/Loss percentages alongside the score in UCI output. |
| `UCI_Opponent` | — | — | — | Opponent info string sent by the GUI (`name elo type`). Used for automatic contempt calculation. |
| `UCI_RatingAdv` | -1000 | -5000 | 5000 | Manually override the rating advantage used for contempt. 0 means automatic based on `UCI_Opponent`. |
| `Contempt` | 300 | -1000 | 1000 | Minimum contempt - minimal scale of the contempt applied to the WDL score. Normally in engine you set hard value of contempt, but in Jackal contempt will scale with the rating of the opponent, but will never go below the set value. |
| `MinimalPrint` | false | — | — | Suppress non-essential info output. |
| `ItersAsNodes` | false | — | — | Report MCTS iterations as nodes instead of cumulative depth. |

## Engine Commands

| Command | Arguments | Description |
| :-- | :-- | :-- |
| `uci` | — | Initializes UCI mode. Prints engine name, author, and available options. |
| `isready` | — | Responds with `readyok` when the engine is ready to receive commands. |
| `ucinewgame` | — | Resets the position and clears the search tree for a new game. |
| `setoption` | `name <name> value <value>` | Sets a UCI option. See Engine Options table above. |
| `position` | `startpos [moves ...]` or `fen <fen> [moves ...]` | Sets the current position from startpos or a FEN string, optionally followed by moves. |
| `go` | `[nodes N]` `[depth N]` `[movetime N]` `[infinite]` `[wtime N]` `[btime N]` `[winc N]` `[binc N]` `[movestogo N]` | Starts the search with the given limits. |
| `stop` | — | Stops the current search. |
| `quit` | — | Exits the engine. |
| `draw` | — | Draws the current board position in the terminal. |
| `eval` | — | Shows a detailed evaluation of the current position, including WDL scores and per-piece values. |
| `policy` | — | Shows policy network output (move probabilities) for all legal moves in the current position. |
| `moves` | — | Lists all legal moves with their policy scores. |
| `tree` | `[depth=1]` `[(half, idx)]` | Draws the MCTS tree from the last search. Optionally start from a specific node. |
| `perft` | `[depth=5]` | Runs a move generation correctness test to the given depth. |
| `bulk` | `[depth=5]` | Runs perft in bulk mode with popcount on the last depth. Faster than regular perft. |
| `bench` | `[depth=5]` | Runs a benchmark on a fixed set of positions. Reports total nodes and NPS. |
| `analyse` | `[nodes=50000]` | Analyses each piece on the board individually using a search per square, showing contribution values. |


## Feature List
* MCTS Search
   * Tree reuse
   * Softmax policy temperature
   * Replacement of least recently used node
   * First play urgency (FPU)
   * CPUCT scaling with depth
   * CPUCT scaling with visit count
   * CPUCT variance scaling
   * Exploration scaling (tau)
   * Gini impurity in exploration
   * Progressive widening
   * Virtual loss for multithreading
   * Proof number search
   * Butterfly history bonus
   * Hashtable with cached value network results
   * Multithreading
   * Sac exploration bonus
   * Sac selection bonus
   * Policy sac bonus
   * Draw pessimism
   * Draw scaling with 50-move rule
   * Contempt based on opponent rating
* Extended Time Manager
   * Soft and hard time limits
   * Phase-based time scaling (more time in middlegame)
   * Game ply scaling (more time in opening)
   * Visit distribution scaling (less time when best move dominates)
   * Visit gap scaling (less time when best move is far ahead in visits)
   * Falling eval extension (more time when score is dropping)
   * Best move instability extension (more time when best move keeps changing)
   * When behind extension (more time when losing)
   * Sacrifice extension (more time when top moves are sacrifices)
* Value Network (×3, base + 2 finetuned)
   * Architecture: `82160 (Threat Inputs) -> 4096 -> 16 -> 128 -> 3`
   * WDL output (win/draw/loss)
   * SCReLU activation on L1, squared ReLU on L2 and L3
   * Horizontal mirroring based on king file
   * Threat Inputs
   * Quantised L0 (int8) and L1 (int16)
   * Network selected based on position score
* Policy Network (×4, base + 3 finetuned)
   * Architecture: `3072 -> 8192 -> 7840 (Move Index)`
   * Dual subnet selection based on SEE result (positive/negative capture)
   * Moving piece output buckets
   * Horizontal mirroring based on king file
   * Quantised L0 (int8) and L1 (int8)
   * Network selected based on position score

## Credits
Jackal is developed by Tomasz Jaworski. Special thanks to:

* [@jw1912](https://github.com/jw1912) for helping me with my previous engine, which allowed me to make Jackal.
* [@jw1912](https://github.com/jw1912) for creating [bullet](https://github.com/jw1912/bullet), that I used for value net training.
* [@Viren6](https://github.com/Viren6) and [@Adam-Kulju](https://github.com/Adam-Kulju) for helping me with concepts of aggressivness in MCTS.
* [@jw1912](https://github.com/jw1912) and [@Viren6](https://github.com/Viren6) for creating [Monty](https://github.com/official-monty/Monty) that provided a lot of help during bugfixing or understanding new ideas.
* [Stefan Pohl](https://www.sp-cc.de) for creating EAS tool, that I'm using to measure progress in aggressivness of Jackal.
* [Vast342](https://github.com/Vast342) for help with quantising value net.
* [Aron Petkovski](https://github.com/aronpetko) for creating MCTS version of butterfly history bonus.
* [rn5f107s2](https://github.com/rn5f107s2) for fixing the multithreading bug.
* [lily](https://github.com/87flowers) for the idea to use proof number search in MCTS.