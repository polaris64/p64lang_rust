# p64lang_rust
Simple language parser, interpreter and CLI built in Rust, to be used for baremetal/no_std environments. 

## Introduction
This repository contains two crates: -

  - p64lang: library containing a parser and interpreter for the work-in-progress P64PL language; and
  - p64lang_cli: binary crate which includes to above library and provides a simple CLI for executing P64PL programs from stdin.

The eventual goal of this project is to create a baremetal/no_std interpreted language for use within another (as yet unreleased) project.  Currently the crates use features of the standard library, but the plan is to replace these features with core alternatives in order to make the parser, interpreter and CLI all no_std.
