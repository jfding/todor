# todor - yet another cli TODO in Rust

## Design Points

- compatibile with my own shell wrapper of mdt'
- in MD, not json
- commands: add, list, edit, count
  - later: shift, sink, collect, postp
  - then: purge, cloudify, tui
  - then: (new) sync with MS-TODO

- config need to go XDG_HOME_CONFIG
- file store at: (default) ~/.local/share/todor/*.md

- cli interface:
  - add : friendly readline
  - list: checklist with hjkl navi key support, <space> to toggle
  - edit: call vi to edit markdown file directly
  - colorful!

- how to parse args: clap
- modules: dirs, colored, cmd_lib, inquire


