module T
  class ActivityWords
    def initialize(options = {})
      @config = options.fetch(:config) { ENV["T_WORDS"] }
    end

    def noun
      "work"
    end

    def present_participle
      "working"
    end

    def past_participle
      "worked"
    end
  end
end
