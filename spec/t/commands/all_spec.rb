require 'spec_helper'
require 'support/command_helpers'

require 't/commands/all'

describe T::Commands::All do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :sparks => %W(0 1 2 3 4 5 6)) }
  include CommandHelpers

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(stdout.string).to eq("") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(stdout.string).to eq("") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-08-01 10:45,2013-08-01 11:15
2013-08-02 10:15,2013-08-02 10:44
2013-08-11 10:45,2013-08-11 11:46
2013-08-22 10:45,2013-08-22 11:47
2013-08-31 10:45,2013-08-31 11:48
2013-09-04 10:45,2013-09-04 11:04
2013-09-04 11:04,2013-09-04 11:16
2013-09-04 11:16,2013-09-04 11:26
2013-09-05 11:26,2013-09-05 11:39
2013-09-05 11:39,2013-09-05 11:49
E_T
      command.run
    end

    it { expect(stdout.string).to eq(<<ALL) }
2013-07-28 - 2013-08-03     59 min    2 segments  min/avg/max/stddev= 29/ 29/ 30/  1  6  6
2013-08-04 - 2013-08-10      0 min
2013-08-11 - 2013-08-17     61 min
2013-08-18 - 2013-08-24     62 min
2013-08-25 - 2013-08-31     63 min
2013-09-01 - 2013-09-07     64 min    5 segments  min/avg/max/stddev= 10/ 12/ 19/  3  643  43
ALL
  end

  context 'with some entries in the file with mixed zones' do
    before do
      File.write(t_file, <<E_T)
2013-08-01 10:45 -0400,2013-08-01 10:15 -0500
2013-08-02 08:15 -0600,2013-08-02 07:44 -0700
2013-08-11 10:45 -0400,2013-08-11 11:46 -0400
2013-08-22 10:45 -0400,2013-08-22 11:47 -0400
2013-08-31 10:45 -0400,2013-08-31 11:48 -0400
2013-09-04 10:45 -0400,2013-09-04 11:04 -0400
2013-09-04 11:04 -0400,2013-09-04 11:16 -0400
2013-09-04 11:16 -0400,2013-09-04 11:26 -0400
2013-09-05 11:26 -0400,2013-09-05 11:39 -0400
2013-09-05 11:39 -0400,2013-09-05 11:49 -0400
E_T
      command.run
    end

    # this might not work right in all time zones :/
    it { expect(stdout.string).to eq(<<ALL) }
2013-07-28 - 2013-08-03     59 min    2 segments  min/avg/max/stddev= 29/ 29/ 30/  1  6  6
2013-08-04 - 2013-08-10      0 min
2013-08-11 - 2013-08-17     61 min
2013-08-18 - 2013-08-24     62 min
2013-08-25 - 2013-08-31     63 min
2013-09-01 - 2013-09-07     64 min    5 segments  min/avg/max/stddev= 10/ 12/ 19/  3  643  43
ALL
  end
end
