<div align="center">

<img
  width="200"
  alt="Jackal Logo"
  src=".readme/logos/logo_rounded_corners.png">
 
<h3>Jackal</h3>
<b>Aggressive MCTS chess engine.</b>
<br>
<br>

[![License](https://img.shields.io/github/license/TomaszJaworski777/Jackal?style=for-the-badge)](https://opensource.org/license/gpl-3-0)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/TomaszJaworski777/Jackal?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/releases/latest)
[![Commits](https://img.shields.io/github/commits-since/TomaszJaworski777/Jackal/latest?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/commits/main)
<br>
<br>

| Version | CCRL 40/15 | CCRL Blitz | Estimated | EAS Score | Release Date |
| :-: | :-: | :-: | :-: | :-: | :-: |
| [1.1.0](https://github.com/TomaszJaworski777/Jackal/releases/tag/1.0.0) | - | - | 3070 | 194.26k | 7th December 2024 |
| [1.0.0](https://github.com/TomaszJaworski777/Jackal/releases/tag/1.0.0) | - | - | 3000 | 229.96k | 4th December 2024 |

</div>

## Overview
Jackal is the chess engine I'm currently developing as a follow-up to [Javelin](https://github.com/TomaszJaworski777/Javelin). It's designed to be a hyper-aggressive engine, still using Monte Carlo Tree Search (MCTS) but with a more focused push toward sharp, tactical play. The idea is to create something that consistently seeks out high-risk, high-reward positions.

I'm building Jackal with everything I learned from developing Javelin, especially around training value and policy neural networks through self-play. It's learning entirely through self-play from noise, but I’ve been adjusting the training process to encourage more aggressive behavior. It also uses my legal move generator, [Spear](https://github.com/TomaszJaworski777/Spear).

## Compiling
1. Download the source code
2. Run `make` command in root folder
3. Binary called `jackal_dev` should appear in the root folder

## Credits
Jackal is developed by Tomasz Jaworski. Special thanks to:

* [@jw1912](https://github.com/jw1912) for helping me with my previous engine, which allowed me to make Jackal.
* [@jw1912](https://github.com/jw1912) for creating [goober](https://github.com/jw1912/goober), that I used for policy training and inference.
* [@jw1912](https://github.com/jw1912) for creating [bullet](https://github.com/jw1912/bullet), that I used for value net training.
* [@Viren6](https://github.com/Viren6) and [@Adam-Kulju](https://github.com/Adam-Kulju) for helping me with concepts of aggressivness in MCTS.
* [@jw1912](https://github.com/jw1912) and [@Viren6](https://github.com/Viren6) for creating [Monty](https://github.com/official-monty/Monty) that provided a lot of help during bugfixing or understanding new ideas.
* [Stefan Pohl](https://www.sp-cc.de) for creating EAS tool, that I'm using to measure progress in aggressivness of Jackal.
* [Vast342](https://github.com/Vast342) for help with quantising value net.

## Command List
Jackal supports all necessary commands to initialize UCI protocol, full description of the protocol can be found [here](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf). Here are additional commands:
* `draw` - Draws the board in the terminal.
* `eval` - Shows evaluation of current position.
* `tree <depth>` - Draws tree of most recent search.
* `perft <depth>` - Runs perft test on current position to specified depth (default = 5).
* `bulk <depth>` - Runs perft test on current position in bulk mode to specified depth (default = 5).
* `moves` - Prints all legal moves together with thier policy.
* `bench <depth>` - Runs bench test to specified depth (default = 5). 

## Feature List
* MCTS Search
   * Tree Reuse
   * Softmax policy temperature at root
   * Replacement of least recently used node
   * First play urgency
   * Scaling C with search duration
   * Exploration scaling
   * Gini impurity in exploration
   * Variance scaling
   * Hashtable with value network results
   * Multithreading
   * Extended time manager
* Value Network
   * Architecture: `768x4->2048->3`
   * WDL net
   * Horizontal mirroring based on kings file
   * Threats and defences inputs
   * Quantised L1 and L2
* Policy Network
   * Architecture: 128 subnet pairs `768->32->32`
   * Selecting subnet pair for move destination based on SEE result

Jackal currently supports multiple threads, but due to some unidentified bug it doesn't play better with more than 1 thread. ($100 for fixing PR)

## EAS
I measured Jackal's EAS to be around 230k, while I also noticed it is very slow to end the games and draws a lot of winning positions. My current guess is that MCTS heavily relies on its neural nets and my current data is just not strong enough to be efficient in end games.

| EAS Score| Sacs | Shorts | Bad draws | Avg. win moves  | Version  |
| :-: | :-: | :-: | :-: | :-: | :-: |
| 194262 | 34.56% | 16.64% | 13.91% | 75 | 1.1.0 |
| 229966 | 40.00% | 26.71% | 17.36% | 103 | 1.0.0 |

List of featues increasing EAS:
- Filtering positions to only those where sides with lower material won the game
- Asymmetrical contempt in PUCT
- Asymmetrucal WDL rescaling

Additional features used in datagen:
- Policy bonus to sacs
- Evaluation bonus to losing materal
