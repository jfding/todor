# Design of routine tasks

## Types of routine tasks
1. daily
2. weekly
3. bi-weekly
4. monthly-on-date
5. monthly-on-weekday (4 weeks)

## How to mark them in markdown
- [d] daily
    - [D] daily (done)
- [w] weekly
    - [W] weekly (done)
- [b] bi-weekly
    - [B] bi-weekly (done)
- [q] monthly-on-weekday
    - [Q] monthly-on-weekday (done)
- [m] monthly-on-date
    - [M] monthly-on-date (done)

## How to operate on them
* normally, routine tasks should be added into "today" or "tomorrow" box, and "INBOX" can have *daily* tasks
* when to run `today` cli with any command, will have a daily-once hook to check the routine tasks in old days boxes, in the logic:
```
check if found belows:
    "daily" in yesterday
    "weekly" in the day of a week ago
    "biweekly" in the day of 2 weeks ago
    "monthly-on-weekday" in the day of 4 weeks ago
    "monthly-on-date" in the day of last monthly
then:
    create a new one(same) in today
    if previous one is done:
        mark it as normal done ([x])
    else:
        mark the found one as normal undone([ ])

```
* for `add`, options are:
  * `-d/--daily`
  * `-w/--weekly`
  * `-b/--biweekly`
  * `-q/--qweekly`
  * `-m/--monthly`
* or we can add it firstly and modify it later by command `edit`
* for `mark` command, just change the routine flag char to upper case
* for `count`, count in the routine ones
* for `import`, will import the matched routine tasks
* for `list` and `listall`, list them with special flag chars
* for `collect` command(INBOX/other -> today), only collect the *daily* tasks, and clear INBOX
* for `pool` command(today -> INBOX), will NOT pooling routine tasks to INBOX
* for `shift` command(today -> tomorrow), will clone a _normal_ task to tomorrow and today unchanged
* [ :sparkles: ] for `sink` command(old days -> today), when scanning old days boxes, in reverse order:
```
if found a routine task:
    if it matched to today:
        create a new item in today
        if done:
            and mark it as normal done ([x])
        else:
            and mark it as normal undone ([ ])

```
* no affect: `purge`, `browse`, `listbox`


