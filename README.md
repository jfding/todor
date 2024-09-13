# todor - yet another cli TODO in Rust

## Design Points

- compatibile with my own shell wrapper of mdt'
- commands: 
  - [x] add
  - [x] list
  - [x] edit
  - [x] count
  - [x] purge
  - [ ] shift
  - [ ] sink
  - [ ] collect
  - [ ] postp
  - [ ] cloudify
  - [ ] sync with MS-TODO

- config need to go XDG_HOME_CONFIG
- in MD, not json
- file store at: (default) ~/.local/share/todor/*.md

- cli interface:
  - add : friendly readline
  - list: checklist with hjkl navi key support, <space> to toggle
  - edit: call vi to edit markdown file directly
  - colorful!
