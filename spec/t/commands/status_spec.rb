require 'spec_helper'
require 'support/command_helpers'

require 't/commands/status'

describe T::Commands::Status do
  subject(:command) { described_class.new(:out => stdout, :file => t_file) }
  include CommandHelpers

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(stdout.string).to eq("NOT working\n") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(stdout.string).to eq("NOT working\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T
      command.run
    end
    it { expect(stdout.string).to eq("NOT working\n") }
  end

  context 'with some incomplete entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T
      command.run
    end
    it { expect(stdout.string).to eq("WORKING\n") }
  end

end
