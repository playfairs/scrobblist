import React from "react";

interface LoadingStateProps {
  message: string;
}

const LoadingState: React.FC<LoadingStateProps> = ({ message }) => {
  return (
    <div className="flex items-center justify-center min-h-screen bg-[#0a0a0a]">
      <div className="text-center">
        <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-[#d51007] mb-4"></div>
        <p className="text-gray-400">{message}</p>
      </div>
    </div>
  );
};

export default LoadingState;
