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
Jackal is my latest chess engine project, built as a follow-up to Javelin. It's a hyper-aggressive engine that leans heavily into Monte Carlo Tree Search (MCTS), much like Javelin, but with a more refined focus on pushing sharp, tactical positions. The idea was to create an engine that plays a very high-risk, high-reward style of chess.

I built Jackal based on the experience I gained from Javelin, especially when it comes to training the value and policy neural networks through self-play. Like Javelin, it learns entirely through self-play, but I’ve tweaked the process to really push it into forcing aggressive play. The self-play setup gives it room to develop its own approach to attack, but it still uses MCTS to evaluate positions. It works smoothly with any UCI-compatible GUI and has been fun to experiment with, especially when watching it launch unexpected, dynamic attacks.

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