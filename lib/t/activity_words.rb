module T
  class ActivityWords
    def initialize(options = {})
      config = options.fetch(:config) { ENV["T_WORDS"] }
      parts = config.to_s.split(":")
      @noun               = parts[0] || "work"
      @past_participle    = parts[1] || (@noun + "ed")
      @present_participle = parts[2] || (@noun + "ing")
    end

    attr_reader :noun, :present_participle, :past_participle
  end
end
