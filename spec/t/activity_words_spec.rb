require "spec_helper"
require "t/activity_words"

describe T::ActivityWords do
  subject { described_class.new(:config => config) }

  context "defaults" do
    let(:config) { nil }
    it { expect(subject.noun).to eq("work") }
    it { expect(subject.past_participle).to eq("worked") }
    it { expect(subject.present_participle).to eq("working") }
  end
end
