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

  context "override with just a noun" do
    let(:config) { "play" }
    it { expect(subject.noun).to eq("play") }
    it { expect(subject.past_participle).to eq("played") }
    it { expect(subject.present_participle).to eq("playing") }
  end

  context "override with a noun and past participle" do
    let(:config) { "play:faked" }
    it { expect(subject.noun).to eq("play") }
    it { expect(subject.past_participle).to eq("faked") }
    it { expect(subject.present_participle).to eq("playing") }
  end

  context "override all the words" do
    let(:config) { "play:faked:sneezing" }
    it { expect(subject.noun).to eq("play") }
    it { expect(subject.past_participle).to eq("faked") }
    it { expect(subject.present_participle).to eq("sneezing") }
  end
end
