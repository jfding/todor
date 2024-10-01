# Design of routine tasks

## Types of routine tasks
1. daily
2. weekly
3. bi-weekly
4. monthly-on-date
5. monthly-on-weekday (4 weeks)
6. outdated

## How to store them in markdown
- [ ] {󰃵:d yyyy-mm-dd} daily
- [ ] {󰃵:w yyyy-mm-dd} weekly
- [ ] {󰃵:b yyyy-mm-dd} bi-weekly
- [ ] {󰃵:q yyyy-mm-dd} monthly-on-weekday
- [ ] {󰃵:m yyyy-mm-dd} monthly-on-date
- [ ] {󰃵:o yyyy-mm-dd} outdated

## Where to save
There will be a dedicated taskbox file, named `ROUTINES.md`, and the structure will be:
```
# ROUTINES

## daily
- [ ] {󰃵:d 2024-08-21} daily task one
- [x] {󰃵:d 2024-08-21} daily task two (done, or stopped)

## weekly
- [ ] {󰃵:w 2024-08-21} weekly task one
- [ ] {󰃵:b 2024-08-21} bi-weekly task two
- [ ] {󰃵:q 2024-08-21} four-weekly task three

## monthly
- [ ] {󰃵:m 2024-08-21} monthly task one

```

## How to operate on them
* use `add` command to add a new routine task with option, and it will be store in ROUTINES md file with proper *routine-prefix* and current date, available options are:
  * `-r/--routine <daily | weekly | biweekly | qweekly | monthly>`
* add a new command `check` to pick up any matched routine tasks to "today" box, still with expanded routine info (details TBD)
* when to run `today` cli with any command, will have a daily-once hook to run `check`
* command `edit` will have a new flag `-r/--routine` to edit the routine tasks
* for `import`, will import the (rarely)matched routine tasks, but with warnings
* for `list` and `listall`, list them with special flags
* for `collect` (INBOX/other -> today) and `postp` (today -> INBOX) will ignore the routine tasks
* for `shift` (today -> tomorrow) and `sink` (old days -> today), will ignore the _daily_ tasks, but move the other routines tasks as _outdated_ tasks
  * but if move from tomorrow back to today, will re-check the matching, if matched then change back to regular routine flags
* no affect: `count`, `mark`, `purge`, `glance`, `listbox`
* (optional) new cmd `routines` to list all the routine tasks
