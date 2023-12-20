## Parspecc(tiv) ##
Name prone to change.

This software package is an addition to Go4 https://github.com/gsi-ee/go4
which helps unpack event-wise experimental data. The package provides streamlined access to
various data members, can perform strong bit-wise checks on each unpacked word.

Inspiration for this project comes from
**Ucesb** http://fy.chalmers.se/~f96hajo/ucesb project, which majority of the
grammar and nomenclature is derived from.

## Structure ##
Every Go4 project starts with the unpack stage where the users are given two interfaces
to expand upon: ``TGo4EventElement`` and ``TGo4EventProcessor``.

Users are then meant to implement their ``Clear()`` and ``BuildEvent()`` methods respectively,
by carefully examining the data structure, matching on certain LMD event/subevent headers, 
detangling payloads from words and sorting them accordingly.

Over time, this accumulates into thousands of lines of overheaded code which can be streamlined 
into custom structures using a specifically built parser. 

# Enter *Parspecc* #
Parspecc declares structures in a separate, .spec file which the lexer and parser examine and
convert into C++ classes. Parsed classes can, and generally will, compose other class instances,
with the main structures being derived from `EVENT(...) {}` blocks, and shall be singletons as a field 
members in the `TGo4EventElement` declaration.

The lexer and parser are written by using the extensive macro system of Rust programming language.

## Prerequisites ##
1. Install Rust:
``
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
``
2. Ready to go



 

