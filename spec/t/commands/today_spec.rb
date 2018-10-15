require 'spec_helper'
require 'support/command_helpers'

require 't/commands/today'

describe T::Commands::Today do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  include CommandHelpers

  before { @now = Time.parse('2013-09-08 13:45:56') }

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(stdout.string).to eq("You have not worked today.\n") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(stdout.string).to eq("You have not worked today.\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T
      command.run
    end
    it { expect(stdout.string).to eq("You have worked for 80 minutes today.\n") }
  end

  context 'with an incomplete entry in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T
      command.run
    end
    it('assumes you worked until now') { expect(stdout.string).to eq("You have worked for 170 minutes today.\n") }
  end

  context 'with an incomplete entry in the file and zones' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},
E_T
      command.run
    end
    it('assumes you worked until now') { expect(stdout.string).to eq("You have worked for 170 minutes today.\n") }
  end

  context 'with some entries in the file from today and yesterday' do
    before do
      File.write(t_file, <<E_T)
2013-09-07 10:45,2013-09-07 11:45
2013-09-07 23:59,2013-09-08 00:01
2013-09-08 11:55,2013-09-08 12:15
E_T
      command.run
    end
    it { expect(stdout.string).to eq("You have worked for 21 minutes today.\n") }
  end
end
