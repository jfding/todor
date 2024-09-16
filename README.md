# todor - yet another cli TODO in Rust

## Design Points

- cli interface design factors:
  - add : friendly readline
  - mark: checklist with hjkl navi key support, <space> to toggle
  - edit: call vi to edit markdown file directly
  - colorful!

- how to store the tasks in fs
  - config need to go XDG_HOME_CONFIG
  - in MD, not json
  - file store at: (default) ~/.local/share/todor/*.md
  - default inbox file will be INBOX.md

## Sub commands

- commands: 
  - [x] add
  - [x] list
  - [x] mark
  - [x] edit
  - [x] count
  - [x] purge
  - [x] glance
  - [x] sink
  - [x] shift
  - [x] collect
  - [x] postp
  - [x] import
  - [x] listbox
  - [ ] sync with iCloud
  - [ ] sync with MS-TODO

