# T

## Installation

    $ gem install t

## Usage

    $ t start
    Starting work.
    $ t stop
    You just worked for 1 minute.
    $ t today
    You have worked for 1 minute today.
    8h=480m
    $ t week
    You have worked for 1 minute since 2013-09-01.
    8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m
    $ t all
    2013-08-25 - 2013-08-31   2200 min
    2013-09-01 - 2013-09-07   1758 min
    8h=480m 16h=960m 24h=1440m 32h=1920m 40h=2400m
    $ t path
    /Users/burke/.t.csv
    $ t edit
    (opens $EDITOR with the csv file)

Your data is stored in `$HOME/.t.csv`.
