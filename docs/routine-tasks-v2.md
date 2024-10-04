# Design of routine tasks

## Types of routine tasks
1. daily
2. weekly
3. bi-weekly
4. monthly-on-date
5. monthly-on-weekday (4 weeks)

## How to store them in markdown
- [d] [yyyy-mm-dd] daily
    - [D] [yyyy-mm-dd] daily (done)
- [w] [yyyy-mm-dd] weekly
    - [W] [yyyy-mm-dd] weekly (done)
- [b] [yyyy-mm-dd] bi-weekly
    - [B] [yyyy-mm-dd] bi-weekly (done)
- [q] [yyyy-mm-dd] monthly-on-weekday
    - [Q] [yyyy-mm-dd] monthly-on-weekday (done)
- [m] [yyyy-mm-dd] monthly-on-date
    - [M] [yyyy-mm-dd] monthly-on-date (done)
- [m] [yyyy-mm-dd] monthly-on-date
    - [M] [yyyy-mm-dd] monthly-on-date (done)
- [o] outdated task
    - [O] outdated task (done)

## Where to save
There will be a dedicated taskbox file, named `ROUTINES.md`, and the structure will be:
```
# ROUTINES

## daily
- [d] [2024-08-21] daily task one
- [D] [2024-08-21] daily task two (done, or stopped)

## weekly
- [w] [2024-08-21] weekly task one
- [b] [2024-08-21] bi-weekly task two
- [q] [2024-08-21] four-weekly task three

## monthly
- [m] [2024-08-21] monthly task one

```

## How to operate on them
* use `add` command to add a new routine task with option, and it will be store in ROUTINES md file with proper flag char and current date, available options are:
  * `-d/--daily`
  * `-w/--weekly`
  * `-b/--biweekly`
  * `-q/--qweekly`
  * `-m/--monthly`
* add a new command `check` to pick up any matched routine tasks to "today" box, still with routine flag chars, to strip original date and append today date as flag `--date` does
* when to run `today` cli with any command, will have a daily-once hook to run `check`
* command `edit` will have a new flag `-r/--routine` to edit the routine tasks
* for `mark` command, just change the routine flag char to upper case, means done for this time
* for `count`, count in the routine ones
* for `import`, will ignore the (exceptional)matched routine tasks
* for `list` and `listall`, list them with special flag chars
* for `collect` (INBOX/other -> today) and `pool` (today -> INBOX) will ignore the routine tasks
* for `shift` (today -> tomorrow) and `sink` (old days -> today), will ignore the _daily_ tasks, but move the other routines tasks as _outdated_ tasks with flag char "o"
  * but if move from tomorrow back to today, will re-check the routine tasks (if not found, still keep [o])
* no affect: `purge`, `glance`, `listbox`
* (optional) new cmd `routines` to list all the routine tasks
