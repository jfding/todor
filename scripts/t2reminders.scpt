#!/usr/bin/env osascript

-- Get the first argument or use default value
on run argv
  if (count of argv) is 0 then
    set boxname to "today"
  else
    set boxname to item 1 of argv
  end if

  tell application "Reminders"
    set todos to do shell script "todor --inbox " & boxname & " list --plain"
    if todos is "" then
      return
    end if

    if not (exists list "Todor") then
      make new list with properties {name:"Todor"}
    end if

    set mylist to list "Todor"

    tell mylist
      set AppleScript's text item delimiters to return
      set todoLines to text items of todos
      repeat with todo in todoLines
        if not (exists reminder todo) then
          make new reminder with properties {name:todo, body:""}
        end if
      end repeat
    end tell
  end tell
end run