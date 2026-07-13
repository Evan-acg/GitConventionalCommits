@echo off
set WORK_DIR="D:\Work\Spaces\Tools\GitConventionalCommits"
set TITLE="AGC"

start "" wt -w -0 -p "Windows PowerShell" -d "%WORK_DIR%" --title "%TITLE%" ^
 ; sp -D -V -p "Windows PowerShell" --size .45 --title "%TITLE%" ^
 ; sp -D -H -p "Windows PowerShell" --size .6 --title "%TITLE%" ^
 ; mf left ^
 ; sp -D -H -p "Windows PowerShell" --size .6 --title "%TITLE%" ^
 ; mf up ^
 ; sp -D -V -p "Windows PowerShell" --size .55 --title "%TITLE%"
 ;
