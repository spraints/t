require 'spec_helper'
require 'support/command_helpers'

require 't/commands/start'

describe T::Commands::Start do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  include CommandHelpers

  before { @now = Time.parse("2013-09-08 13:45:56 #{tz_offset}") }

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(File.read(t_file)).to eq("2013-09-08 13:45 #{tz_offset},\n") }
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(File.read(t_file)).to eq("2013-09-08 13:45 #{tz_offset},\n") }
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},2013-09-08 12:15 #{tz_offset}
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},2013-09-08 12:15 #{tz_offset}
2013-09-08 13:45 #{tz_offset},
E_T
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with an incomplete entry in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},
E_T
    it { expect(stdout.string).to eq("You already started working, 110 minutes ago!\n") }
  end
end
