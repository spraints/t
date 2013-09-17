require 'spec_helper'
require 'support/command_helpers'

require 't/commands/all'

describe T::Commands::All do
  subject(:command) { described_class.new(:out => stdout, :file => t_file) }
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
2013-09-04 10:45,2013-09-04 11:49
E_T
      command.run
    end

    it { expect(stdout.string).to eq(<<ALL) }
2013-07-28 - 2013-08-03     59 min    2 segments  min/max/avg=29/30/29 min
2013-08-04 - 2013-08-10      0 min
2013-08-11 - 2013-08-17     61 min
2013-08-18 - 2013-08-24     62 min
2013-08-25 - 2013-08-31     63 min
2013-09-01 - 2013-09-07     64 min
ALL
  end
end
