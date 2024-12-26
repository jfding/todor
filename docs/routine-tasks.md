# Design of routine tasks

## Types of routine tasks
1. daily
2. weekly
3. bi-weekly
4. monthly-on-date
5. monthly-on-weekday (4 weeks)
6. once (one-shot reminder)

## How to store them in markdown
- [ ] {󰃵:d yyyy-mm-dd} daily
- [ ] {󰃵:w yyyy-mm-dd} weekly
- [ ] {󰃵:b yyyy-mm-dd} bi-weekly
- [ ] {󰃵:q yyyy-mm-dd} monthly-on-weekday
- [ ] {󰃵:m yyyy-mm-dd} monthly-on-date
- [ ] {󰃵:1 yyyy-mm-dd} reminder

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
  * `-r/--routine <daily | weekly | biweekly | qweekly | monthly | once>`
* add a new command `checkout` to pick up any matched routine tasks to "today" box, with expanded routine info (means "checkout")
  * actually its an alias of `collect --inbox routines`
* when to run `today` cli with any command, will have a daily-once hook to run `checkout`
* command `edit` will have a new flag `-r/--routine` to edit the routine tasks
* for `import`, will import the (rarely)matched routine tasks to "ROUTINES" box
* for `list` and `listall`, list them with special flags
  * new cmd `routines` to list all the routine tasks
* cmd `pool` (today -> INBOX) will ignore the checkout routine tasks
* cmd `collect` (INBOX/other -> today) will only checkout routine tasks from ROUTINES box to "today" box, otherwise just move without checkout operation
  * and `collect --inbox routines` is dedicated for routine tasks checkout
* no affect: `count`, `mark`, `purge`, `browse`, `listbox`, `shift`, `sink`
