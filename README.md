<div align="center">

<img
  width="200"
  alt="Jackal Logo"
  src=".readme/logos/logo.png">
 
<h3>Jackal</h3>
<b>Most aggressive MCTS chess engine.</b>
<br>
<br>

[![License](https://img.shields.io/github/license/TomaszJaworski777/Jackal?style=for-the-badge)](https://opensource.org/license/gpl-3-0)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/TomaszJaworski777/Jackal?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/releases/latest)
[![Commits](https://img.shields.io/github/commits-since/TomaszJaworski777/Jackal/latest?style=for-the-badge)](https://github.com/TomaszJaworski777/Jackal/commits/main)
<br>
<br>

| Version | CCRL 40/15 | CCRL Blitz | Estimated | EAS Score | Release Date |
| :-: | :-: | :-: | :-: | :-: |

</div>

## Overview
ackal is the chess engine I'm currently developing as a follow-up to Javelin. It's designed to be a hyper-aggressive engine, still using Monte Carlo Tree Search (MCTS) but with a more focused push toward sharp, tactical play. The idea is to create something that consistently seeks out high-risk, high-reward positions.

I'm building Jackal with everything I learned from developing Javelin, especially around training value and policy neural networks through self-play. Like Javelin, it's learning entirely through self-play, but I’ve been adjusting the training process to encourage more aggressive behavior. It also uses my legal move generator, [Spear](https://github.com/TomaszJaworski777/Spear).

## Compiling

## Credits
Jackal is developed by Tomasz Jaworski. Special thanks to:

* [@jw1912](https://github.com/jw1912) for helping me with my previous engine, which allowed me to make Jackal.
* [@jw1912](https://github.com/jw1912) for creating [goober](https://github.com/jw1912/goober), that I used for policy training and inference.
* [@jw1912](https://github.com/jw1912) for creating [bullet](https://github.com/jw1912/bullet), that I used for value net training.

## Command List
Jackal supports all necessary commands to initialize UCI protocol, full description of the protocol can be found [here](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf). Here are additional commands:
* `draw` - Draws the board in the terminal.
* `tree <depth>` - Draws tree of most recent search.
* `tree <depth> <node>` - Draws tree of most recent search from provided node index.
* `perft <depth>` - Runs perft test on current position.
* `bulk <depth>` - Runs perft test on current position in bulk mode.
* `bench <depth>` - Runs benchmark to test engine speed.

## Feature List