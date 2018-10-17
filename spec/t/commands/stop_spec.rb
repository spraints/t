require 'spec_helper'
require 'support/command_helpers'

require 't/commands/stop'

require 'time'

describe T::Commands::Stop do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  include CommandHelpers

  before { @now = Time.parse('2013-09-08 13:45:56') }

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(File.exists?(t_file)).to be_falsey }
    it { expect(stdout.string).to eq("You haven't started working yet!\n") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(File.read(t_file)).to eq("") }
    it { expect(stdout.string).to eq("You haven't started working yet!\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T
    it { expect(stdout.string).to eq("You haven't started working yet!\n") }
  end

  context 'with a started entry in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 13:45 #{tz_offset}
E_T
    it { expect(stdout.string).to eq("You just worked for 110 minutes.\n") }
  end

  context 'with a started entry in the file, no zones' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45 #{tz_offset},2013-09-08 11:45 #{tz_offset}
2013-09-08 11:55 #{tz_offset},2013-09-08 13:45 #{tz_offset}
E_T
    it { expect(stdout.string).to eq("You just worked for 110 minutes.\n") }
  end

  context 'with multiple started entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,
2013-09-08 11:55,
E_T
      command.run
    end
    it { expect(stdout.string).to eq("Your file has more than one session started. Please `t edit` to fix it.\n") }
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45,
2013-09-08 11:55,
E_T
  end
end
