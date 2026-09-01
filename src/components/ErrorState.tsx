import React from "react";

interface ErrorStateProps {
  message: string;
  onRetry: () => void;
}

const ErrorState: React.FC<ErrorStateProps> = ({ message, onRetry }) => {
  return (
    <div className="flex items-center justify-center min-h-screen bg-[#0a0a0a]">
      <div className="text-center max-w-md">
        <div className="text-red-500 text-4xl mb-4">⚠</div>
        <h2 className="text-xl font-bold text-white mb-2">Error</h2>
        <p className="text-gray-400 mb-6">{message}</p>
        <button
          onClick={onRetry}
          className="px-6 py-3 bg-[#d51007] hover:bg-[#b50d05] text-white rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );
};

export default ErrorState;
