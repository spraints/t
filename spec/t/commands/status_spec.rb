require 'spec_helper'
require 'support/command_helpers'

require 't/commands/status'

describe T::Commands::Status do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  include CommandHelpers

  let(:time_stub) { double("Time").tap { |t| allow(t).to receive(:now) { now } } }
  let(:now) { Time.parse("2013-09-08 12:20:00") }

  context 'with no file' do
    before { File.unlink t_file }

    example do
      command.run
      expect(stdout.string).to eq("NOT working\n")
    end

    example do
      command.run("--with-week")
      expect(stdout.string).to eq("NOT working (0)\n")
    end
  end

  context 'with an empty file' do
    example do
      command.run
      expect(stdout.string).to eq("NOT working\n")
    end

    example do
      command.run("--with-week")
      expect(stdout.string).to eq("NOT working (0)\n")
    end
  end

  context 'with some entries in the file' do
    before { File.write(t_file, <<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T

    example do
      command.run
      expect(stdout.string).to eq("NOT working\n")
    end

    example do
      command.run("--with-week")
      expect(stdout.string).to eq("NOT working (80)\n")
    end
  end

  context 'with some incomplete entries in the file' do
    before { File.write(t_file, <<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T

    example do
      command.run
      expect(stdout.string).to eq("WORKING\n")
    end

    example do
      command.run("--with-week")
      expect(stdout.string).to eq("WORKING (85)\n")
    end
  end

end
