# todor - Todo Cli Tools in Rust "(yet another) TODO cli in Rust"

## Design Points

- compatibile with my own shell wrapper of mdt'
- in MD, not json
- commands: add, list, edit, count
  - later: shift, sink, collect, postp
  - then: purge, cloudify, tui
  - then: (new) sync with MS-TODO

- config need to go XDG_HOME_CONFIG
- file store at: (default) ~/.local/state/todor/*.md

- cli interface:
  - add : friendly readline
  - list: checklist with hjkl navi key support, <space> to toggle
  - colorful!

- how to parse args: start with the basic env::args, then clap
- modules: dirs, colored, rustyline, cmd_lib, inquire
- #[no_link]


