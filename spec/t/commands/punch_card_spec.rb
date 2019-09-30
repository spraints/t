require 'spec_helper'
require 'support/command_helpers'

require 't/commands/punch_card'

describe T::Commands::PunchCard do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :zero => "0", :sparks => %W(1 2 3 4), :cols => cols) }
  let(:cols) { T::Commands::PunchCard::LineHeaderWidth + 7 + 24*7 } # one column per hour of the week
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
2013-09-04 11:16,2013-09-04 11:59
2013-09-05 11:26,2013-09-05 11:39
2013-09-05 11:39,2013-09-05 11:49
E_T
      command.run
    end

    let(:lines) { stdout.string.split("\n") }
    it { expect(lines.size).to eq(6) }
    let(:empty_day) { "0"*24 }
    it { expect(lines[0]).to eq "2013-07-28 - 2013-08-03     59 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000011000000000000|000000000020000000000000|#{empty_day}|" }
    it { expect(lines[1]).to eq "2013-08-04 - 2013-08-10      0 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|" }
    it { expect(lines[2]).to eq "2013-08-11 - 2013-08-17     61 min|000000000013000000000000|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|" }
    it { expect(lines[3]).to eq "2013-08-18 - 2013-08-24     62 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000013000000000000|#{empty_day}|#{empty_day}|" }
    it { expect(lines[4]).to eq "2013-08-25 - 2013-08-31     63 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000013000000000000|" }
    it { expect(lines[5]).to eq "2013-09-01 - 2013-09-07    107 min|#{empty_day}|#{empty_day}|#{empty_day}|000000000014000000000000|000000000002000000000000|#{empty_day}|#{empty_day}|" }
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
2013-09-04 11:16 -0400,2013-09-04 11:59 -0400
2013-09-05 11:26 -0400,2013-09-05 11:39 -0400
2013-09-05 11:39 -0400,2013-09-05 11:49 -0400
E_T
      command.run
    end

    let(:lines) { stdout.string.split("\n") }
    it { expect(lines.size).to eq(6) }
    let(:empty_day) { "0"*24 }
    it { expect(lines[0]).to eq "2013-07-28 - 2013-08-03     59 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000011000000000000|000000000020000000000000|#{empty_day}|" }
    it { expect(lines[1]).to eq "2013-08-04 - 2013-08-10      0 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|" }
    it { expect(lines[2]).to eq "2013-08-11 - 2013-08-17     61 min|000000000013000000000000|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|" }
    it { expect(lines[3]).to eq "2013-08-18 - 2013-08-24     62 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000013000000000000|#{empty_day}|#{empty_day}|" }
    it { expect(lines[4]).to eq "2013-08-25 - 2013-08-31     63 min|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|#{empty_day}|000000000013000000000000|" }
    it { expect(lines[5]).to eq "2013-09-01 - 2013-09-07    107 min|#{empty_day}|#{empty_day}|#{empty_day}|000000000014000000000000|000000000002000000000000|#{empty_day}|#{empty_day}|" }
  end
end
