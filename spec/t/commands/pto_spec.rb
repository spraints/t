require 'spec_helper'
require 'support/command_helpers'

require 't/commands/pto'

describe T::Commands::Pto do
  subject(:command) { described_class.new(:out => stdout, :file => t_file) }
  include CommandHelpers

  let(:argv) { [] }
  def run
    command.run *argv
  end

  context 'with no file' do
    before do
      File.unlink t_file
      run
    end
    it { expect(stdout.string).to eq("") }
  end

  context 'with an empty file' do
    before do
      run
    end
    it { expect(stdout.string).to eq("") }
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
      run
    end

    it { expect(stdout.string).to match(/^2013-08-25 work=  60 pto=2340\n/) }
    it { expect(stdout.string).to match(/^2013-09-01 work= 240 pto=2160\n/) }
    it { expect(stdout.string).to match(/^2013 total_pto= 4500/) }

    context "expected work week is 200 minutes" do
      let(:argv) { ["200"] }
      it { expect(stdout.string).to match(/^2013-08-25 work=  60 pto= 140\n/) }
      it { expect(stdout.string).to match(/^2013-09-01 work= 240 pto=   0\n/) }
      it { expect(stdout.string).to match(/^2013 total_pto=  140/) }
    end
  end

  context 'with some entries in the file with mixed zones' do
    before do
      File.write(t_file, <<E_T)
2013-08-31 10:45 -0400,2013-08-31 10:45 -0500
2013-09-01 08:45 -0600,2013-09-01 08:45 -0700
2013-09-02 10:45 -0400,2013-09-02 11:45 -0400
2013-09-03 10:45 -0400,2013-09-03 11:45 -0400
2013-09-04 10:45 -0400,2013-09-04 11:45 -0400
E_T
      run
    end

    it { expect(stdout.string).to match(/^2013-08-25 work=  60 pto=2340\n/) }
    it { expect(stdout.string).to match(/^2013-09-01 work= 240 pto=2160\n/) }
    it { expect(stdout.string).to match(/^2013 total_pto= 4500/) }

    context "expected work week is 200 minutes" do
      let(:argv) { ["200"] }
      it { expect(stdout.string).to match(/^2013-08-25 work=  60 pto= 140\n/) }
      it { expect(stdout.string).to match(/^2013-09-01 work= 240 pto=   0\n/) }
      it { expect(stdout.string).to match(/^2013 total_pto=  140/) }
    end
  end

  context 'spanning a week boundary' do
    before do
      File.write(t_file, <<E_T)
2013-08-31 23:30,2013-09-01 00:45
E_T
      run
    end

    it { expect(stdout.string).to match(/2013-08-25 work=  30 /) }
    it { expect(stdout.string).to match(/2013-09-01 work=  45 /) }
  end
end
