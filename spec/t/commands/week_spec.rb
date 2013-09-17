require 'spec_helper'
require 'support/command_helpers'

require 't/commands/week'

describe T::Commands::Week do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  include CommandHelpers

  context 'with no file' do
    before do
      File.unlink t_file
      @now = Time.parse("2013-09-07 23:59:59") # The last second of the week of 9/1.
      command.run
    end
    it { expect(stdout.string).to eq("You have not worked since 2013-09-01.\n") }
  end

  context 'with an empty file' do
    before do
      @now = Time.parse("2013-09-07 23:59:59") # The last second of the week of 9/1.
      command.run
    end
    it { expect(stdout.string).to eq("You have not worked since 2013-09-01.\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-08-31 10:45,2013-08-31 11:45
2013-09-01 10:45,2013-09-01 11:45
2013-09-02 10:45,2013-09-02 11:45
2013-09-03 10:45,2013-09-03 11:45
2013-09-04 10:45,2013-09-04 11:45
E_T
      @now = Time.parse(now_for_this_test)
      command.run
    end

    context 'today is Sunday' do
      let(:now_for_this_test) { "2013-09-01 00:00:00" }
      it { expect(stdout.string).to eq("You have worked for 240 minutes since 2013-09-01.\n") }
    end

    context 'today is Wednesday' do
      let(:now_for_this_test) { "2013-09-04 00:00:00" }
      it { expect(stdout.string).to eq("You have worked for 240 minutes since 2013-09-01.\n") }
    end

    context 'today is Saturday' do
      let(:now_for_this_test) { "2013-09-07 23:59:59" }
      it { expect(stdout.string).to eq("You have worked for 240 minutes since 2013-09-01.\n") }
    end

    context 'today is next Sunday' do
      let(:now_for_this_test) { "2013-09-08 00:00:00" }
      it { expect(stdout.string).to eq("You have not worked since 2013-09-08.\n") }
    end
  end
end
